<!--
    Written by GPT-5.1. This design is heavily influenced by Go's runtime heap allocator.
--->

# Dust GC / Heap Library Design Summary

This document describes the design of the runtime heap and allocation system used by the library,
including the relationships between structures, concurrency/locking model, and allocation flow. It
is written to be self‑contained and does not rely on external context.

---

                  ┌───────────────────────────────────────────────────────────────┐
                  │                               VM                              │
                  └───────────────────────────────────────────────────────────────┘
                                          │
                                          │  owns
                                          ▼
                  ┌───────────────────────────────────────────────────────────────┐
                  │                           GlobalHeap                          │
                  └───────────────────────────────────────────────────────────────┘
                     │                     │                          │
                     │                     │                          │
                     │                     │                          │
                     ▼                     ▼                          ▼
        ┌─────────────────────┐   ┌─────────────────────┐   ┌─────────────────────┐
        │    PageAllocator    │   │    BlockCache[]     │   │       Arena[]       │
        └─────────────────────┘   └─────────────────────┘   └─────────────────────┘
                  │                     │                          │
                  │                     │                          │
                  │                     │                          │
                  ▼                     ▼                          ▼
        ┌─────────────────────┐   ┌─────────────────────┐   ┌─────────────────────┐
        │    RadixTree /      │   │   BlockCache for    │   │        Arena        │
        │   page bitmaps      │   │   SpanClass = k     │   │ (base + MmapMut)    │
        └─────────────────────┘   └─────────────────────┘   └─────────────────────┘
                                       │
                                       │
          ┌────────────────────────────┼────────────────────────────┐
          ▼                            ▼                            ▼
┌─────────────────────┐      ┌─────────────────────┐      ┌─────────────────────┐
│  partial_swept list │      │ partial_unswept list│      │  full_* lists ...   │
└─────────────────────┘      └─────────────────────┘      └─────────────────────┘
          │                            │                            │
          │                            │                            │
          ▼                            ▼                            ▼
      Block ─── Block ─── …        Block ─── Block ─── …        Block ─── Block ─── …



                  ┌───────────────────────────────────────────────────────────────┐
                  │                          ThreadHeap                           │
                  └───────────────────────────────────────────────────────────────┘
                                          │
                                          │  has parent (Arc<GlobalHeap>)
                                          ▼
                  ┌───────────────────────────────────────────────────────────────┐
                  │                           GlobalHeap                          │
                  └───────────────────────────────────────────────────────────────┘

                  ┌───────────────────────────────────────────────────────────────┐
                  │                          ThreadHeap                           │
                  └───────────────────────────────────────────────────────────────┘
                     │                     │                       │
                     │                     │                       │
                     ▼                     ▼                       ▼
        ┌─────────────────────┐   ┌─────────────────────┐   ┌─────────────────────┐
        │  blocks[SpanClass]  │   │      tiny state     │   │     PageCache       │
        └─────────────────────┘   └─────────────────────┘   └─────────────────────┘
                     │
                     │  for SpanClass = k
                     ▼
        ┌─────────────────────┐
        │   Option<Block*>    │  (current block for class k)
        └─────────────────────┘
                     │
                     │  points into
                     ▼
        Block ────────────────────────────────────────────────────────────────► slots / objects



   Arena / page layout (one Arena):

   base address
        │
        ▼
   ┌───────────────┬───────────────┬───────────────┬───────────────┬───────┐
   │    page 0     │    page 1     │    page 2     │    page 3     │  ...  │
   └───────────────┴───────────────┴───────────────┴───────────────┴───────┘
        │                 │
        │   contiguous    │
        └──── pages ──────┘
                 ▲
                 │
                 │   Block covering N pages
                 │
   ┌───────────────────────────────────────────────────────────────────────────┐
   │                                   Block                                  │
   └───────────────────────────────────────────────────────────────────────────┘



   Block internal layout (fixed-size slots):

   start of Block
        │
        ▼
   ┌─────────┬─────────┬─────────┬─────────┬─────────┬───────┐
   │ slot 0  │ slot 1  │ slot 2  │ slot 3  │ slot 4  │  ...  │
   └─────────┴─────────┴─────────┴─────────┴─────────┴───────┘
        ▲
        │
        │  individual allocations (objects)
        ▼
      object



   Relationships between lists and Blocks:

   BlockCache for one SpanClass:

       partial_swept:
           head
            │
            ▼
        ┌─────────┐    ┌─────────┐    ┌─────────┐
        │ Block A │───►│ Block B │───►│ Block C │
        └─────────┘◄───└─────────┘◄───└─────────┘
            ▲                           ▲
            │                           │
            └───────── intrusive linked ┘
                      via Block.next / Block.prev


   ThreadHeap points to a single Block from such a list:

       ThreadHeap.blocks[k]  ─────────────►  Block B (taken from BlockCache list)


   Global page index / arenas (conceptual):

       pages: 0 ............................................. N

       [ 0 .............. X )     → Arena 0 pages
       [ X .............. Y )     → Arena 1 pages
       [ Y .............. Z )     → Arena 2 pages
       ...

       PageAllocator / RadixTree operates over this 0..N index space; each index
       maps down into a specific Arena and page within that Arena.



## 1. High-Level Structure

The library implements a page-based heap allocator with:

- A **single heap instance per VM** (`GlobalHeap`), owning:
  - A page allocator and arena map.
  - A set of per–size‑class central caches.
- A set of **per-thread allocators** (`ThreadHeap`), each tied to exactly one `GlobalHeap`.
- **Blocks** of contiguous pages, each subdivided into fixed-size slots for small objects, with
  metadata for allocation and GC.
- A **page allocator** with:
  - A bitmap and summary tree tracking free vs used pages.
  - Arena metadata for mapping addresses to spans/blocks.

The design is:

- Page-based: the fundamental unit of the underlying allocator is a page, and blocks are contiguous
  runs of pages.
- Size-class-based: small allocations are rounded to one of a fixed set of size classes, each with
  its own block layout and central cache.
- Per-thread caching: each thread has a `ThreadHeap` with cached blocks and a local page cache to
  reduce contention and global coordination.

The structures are designed to allow the VM to create and destroy heaps freely, with no reliance on
process-global singletons. Each VM instance owns exactly one `GlobalHeap`, and each `ThreadHeap`
holds an explicit reference to its parent `GlobalHeap`.

---

## 2. GlobalHeap

### 2.1 Responsibility

`GlobalHeap` is the root allocator for a VM instance. It is responsible for:

- **Managing arenas**:
  - Maintaining a set of arenas, each a large contiguous region of virtual memory mapped from the
    OS.
  - Tracking which arenas exist and what address ranges they cover.
- **Page allocation and reclamation**:
  - Managing a page allocator that operates over the address space of arenas.
  - Keeping track of which pages are free, in use, or scavenged, and supporting growth when needed.
- **Central caches per span class**:
  - Maintaining one central cache per span/size class, each of which manages blocks (spans) for that
    class.
- **Providing slow-path allocation services**:
  - Supplying new blocks to `ThreadHeap` instances when their current blocks are exhausted.
  - Allocating and freeing blocks at the page level as requested by central caches.
- **Accounting and statistics**:
  - Tracking total bytes/pages allocated and freed at the page/block level.
  - Optionally exposing these metrics for diagnostics or tuning.

### 2.2 Fields

`GlobalHeap` contains:

- A **page allocator**, protected by a mutex:
  - The page allocator manages the underlying physical/virtual memory at the granularity of pages.
  - All page-level operations (allocating and freeing page ranges, growing arenas) go through this
    mutex.
- An array of **central block caches**, one per span class:
  - Each element is a central cache for a specific span class.
  - Each cache is protected by its own mutex, allowing independent locking at the span-class level.
- A vector of **arenas**:
  - Each arena is a fixed-size region of virtual memory (e.g., 64 MiB) mapped from the OS.
  - Arenas are used by the page allocator, which indexes pages across all arenas.
- **Arena hints**:
  - A structure holding candidate address ranges or indices where new arenas should be created when
    the heap grows.
  - Used to select addresses for new arena mappings in a way that keeps the heap layout reasonable
    and somewhat contiguous.

### 2.3 Concurrency and Locking

`GlobalHeap` uses fine-grained locking:

- **Per-central locks**:
  - Each central block cache has its own mutex.
  - Allocating or freeing blocks through a central only requires taking the lock for the relevant
    span class.
- **Heap/page allocator lock**:
  - The page allocator is behind a separate mutex.
  - All operations that find or mark free page ranges, grow the heap, or integrate new arena memory
    use this lock.

Lock ordering is strictly defined to avoid deadlocks:

1. A central cache mutex may be locked first.
2. While holding a central cache lock, the page allocator mutex may be acquired as needed for
   underlying page operations.
3. No code is allowed to hold the page allocator mutex and then attempt to lock a central cache.

This enforces a consistent global ordering: **central → heap allocator**, never the reverse.

### 2.4 Lifetime and Ownership

- Each VM instance owns a `GlobalHeap` value, typically wrapped in a reference-counted pointer so
  multiple threads can share it.
- Each `ThreadHeap` holds an owned reference to its parent `GlobalHeap`, establishing an explicit
  parent-child relationship.
- When a VM is destroyed:
  - The `GlobalHeap` is dropped after all `ThreadHeap` references to it have been released.
  - All heap data structures (arenas, page allocator state, central caches) can be freed, and OS
    mappings unmapped, as appropriate.

This design ensures that no static global heap is required. Multiple VMs can coexist in a single
process, each with its own independent heap.

---

## 3. ThreadHeap

### 3.1 Responsibility

`ThreadHeap` represents per-thread allocation state, analogous to a thread-local or P-local
allocator cache. Its responsibilities are:

- Providing **fast, uncontended allocation paths** for small objects:
  - For each span class, it may hold a currently active block (span) from which it allocates slots.
- Implementing a **tiny allocator** for very small objects:
  - Coalesces multiple tiny allocations into a single slot or page region, reducing overhead for
    very small sizes.
- Maintaining a **per-thread page cache**:
  - Caches small numbers of pages or page ranges for efficient page-level operations without hitting
    the global page allocator for every request.
- Managing a **scratch block pointer** for temporary use when returning or acquiring blocks from the
  central caches.

`ThreadHeap` is the primary entry point for object allocation. The interpreter or JIT will call
`ThreadHeap`’s allocation methods, which perform fast-path allocation and use `GlobalHeap`/central
caches on slow paths.

### 3.2 Fields

`ThreadHeap` contains:

- A reference to its parent `GlobalHeap`:
  - Owned (e.g., through a reference-counting pointer) so that the heap remains alive as long as any
    `ThreadHeap` using it exists.
  - Enforces that each `ThreadHeap` is permanently associated with exactly one `GlobalHeap`.
- An array of **current blocks** indexed by span class:
  - Each entry may hold one block currently being allocated from for that span class.
  - Allocation from these blocks is done without locking.
- **Tiny allocation state**:
  - A pointer to a tiny allocation region (if any), plus an offset and a count of how many tiny
    objects have been allocated from it.
  - Used by tiny allocation routines to pack small objects into a minimal number of slots.
- A per-thread **page cache**:
  - A small cache for page allocations, typically represented as a base index and a compact bitmap
    or word describing available pages.
  - Used in conjunction with the underlying page allocator to reduce contention.
- A **free block scratch**:
  - A field reserved for temporarily storing a block when returning or acquiring from central
    caches.
  - Allows reusing a block or holding it briefly without immediately returning it to the central.

### 3.3 Allocation Flow

A typical small allocation proceeds as follows:

1. The caller invokes the appropriate allocation method on `ThreadHeap` with the requested size and
   a flag indicating whether the object may contain pointers.
2. The allocation method:
   - Determines the appropriate size class for the requested size.
   - Computes the span class (size class + “scan” vs “noscan”) for the allocation.
   - Looks up the current block for that span class in the `alloc` array.
3. If the current block:
   - Has free slots, it allocates directly from that block without any locks or global coordination.
   - Has no free slots, or if no current block exists for that span class, the method takes a slow
     path to obtain a new block from a central cache.

On the slow path:

1. The method computes the index for the span class.
2. It uses the reference to `GlobalHeap` to lock the central block cache for that span class (via
   the central’s mutex).
3. It asks the central cache to provide a block, potentially using the per-thread page cache and, if
   necessary, calling into the page allocator within `GlobalHeap`.
4. Once a block is obtained, the method stores it into the `alloc` array and then retries the
   allocation from the new block.

All of this is hidden behind `ThreadHeap` methods so that the interpreter or JIT-generated code only
needs to know:

- How to compute the size/span class for a given allocation.
- How to call the appropriate allocation method on `ThreadHeap`.

### 3.4 Entry Point and Ownership

- `ThreadHeap` is constructed with a reference to a `GlobalHeap`, establishing its parent.
- It is not meant to be reused with different `GlobalHeap` instances; a `ThreadHeap` is permanently
  bound to the heap from which it was created.
- The combination of `GlobalHeap` (owned by the VM) and multiple `ThreadHeap` instances (owned by
  threads or execution contexts) mirrors the relationship between a global heap and local allocation
  caches.

---

## 4. Block (Span) Structure

### 4.1 Responsibility

A **block** represents a contiguous run of pages that is subdivided into fixed-size slots
corresponding to a particular size class and span class. Its responsibilities include:

- Mapping from an address within the block to a slot index.
- Tracking which slots are allocated vs free.
- Providing a fast “next free” index to support quick allocation.
- Holding optional GC metadata for precise or conservative scanning.
- Linking into various lists maintained by central caches and the page allocator.

### 4.2 Fields and Invariants

A block stores:

- **Start address and page count**:
  - The base address of the first byte in the block.
  - The number of pages that the block spans.
- **Span class**:
  - Encodes both the size class and whether objects contain pointers (scan vs noscan).
- **Size-class metadata**:
  - The size class used for this block.
  - The size (in bytes) of each slot.
  - The number of slots in the block.
- **Allocation bitmap and counters**:
  - A bit vector indicating which slots are allocated.
  - A free index indicating where to begin scanning for the next free slot.
  - A count of currently allocated slots.
- **GC metadata**:
  - Optional per-slot “heap bits” or mark bits used by the collector to know where pointers are and
    which objects are live.
  - Whether a separate allocation header is needed for large objects with pointers.
- **Linked-list pointers**:
  - `next` and `prev` pointers used for intrusive linked lists in central caches and/or other global
    lists.
  - These pointers are intended to be used for exactly one owning list at a time (e.g., a central
    cache list or a free-block pool).

Invariants:

- The span class of a block determines its size class and scanning policy.
- All slots in a block have the same size and pointer-layout rules.
- A block may be in one of several logical states (free, partially full, full), and central caches
  maintain separate collections or queues for blocks in these states.

---

## 5. BlockCache (Central Cache) Structure

### 5.1 Responsibility

A **block cache** is a central cache for a specific span class. Its main responsibilities are:

- Managing blocks (spans) for one span class:
  - Keeping partially used blocks available for replenishing thread-local allocators.
  - Tracking fully used blocks that may later become available again.
  - Acting as the intermediate layer between `ThreadHeap` and the global page allocator.
- Enforcing concurrency and coordination:
  - Providing a serialized interface for acquiring and returning blocks among many threads.
- Interacting with the global page allocator:
  - Requesting new blocks of pages when the cache is empty or exhausted.
  - Returning fully free blocks to the page allocator when they become unused.

### 5.2 Fields

Each block cache is associated with exactly one span class and contains:

- A span class identifier.
- One or more collections (possibly separated) for combinations of:
  - Partial vs full blocks.
  - Swept vs unswept blocks for GC purposes.

Internally, these collections may be:

- Ring buffers, lists, or queues.
- Intrusive linked lists using the `next`/`prev` pointers on each block.
- Implemented with or without extra locking, depending on the concurrency model.

The entire block cache is protected by a mutex, ensuring that:

- Acquiring or returning a block is atomic with respect to other threads.
- The internal lists or buffers can be updated safely without per-element atomic operations.

### 5.3 Interaction with GlobalHeap and PageAllocator

When a `ThreadHeap` needs a new block for a span class:

1. It locks the corresponding block cache using the mutex stored in `GlobalHeap`.
2. It requests a block from the block cache.

The block cache then:

1. Attempts to serve the request from its partial or full lists (depending on policy):
   - It may prioritize partially filled and already-swept blocks.
   - It may perform limited work to sweep or reclaim unswept blocks if needed.
2. If no suitable block is available, it requests a new block from the underlying page allocator:
   - This is done by using the `GlobalHeap` reference to lock the page allocator and allocate a new
     run of pages for this span class.
   - The new block is initialized with the correct size class, slot count, and allocation/GC
     metadata.

When blocks are freed (e.g., becaused they have been swept and become entirely empty):

- The block cache may return them to the page allocator using similar interactions:
  - Lock the page allocator.
  - Mark the pages as free.
  - Release or recycle the block’s metadata as appropriate.

All of these operations occur under the block cache mutex, then optionally under the page allocator
mutex, preserving the lock ordering rule.

---

## 6. PageAllocator and RadixTree

### 6.1 Responsibility

The **page allocator** manages free and used pages across the entire heap address space. It is
responsible for:

- Tracking which pages are free vs allocated.
- Finding contiguous runs of pages for new blocks.
- Returning freed pages to the free pool.
- Growing the heap by adding new arenas when needed.

It uses a radix-tree-like structure, backed by bitmaps and summary values, to efficiently search for
free runs of pages.

### 6.2 Data Structures

The page allocator includes:

- A bitmap or multi-level structure where:
  - Leaf nodes represent contiguous ranges of pages with a compact bitmap indicating free/used
    status.
  - Internal nodes store summaries describing the free capacity in their subtree:
    - For each node, a compact summary indicates:
      - The length of the longest free run.
      - Where that run starts and ends within the node’s range.
- A search position:
  - A global “search address” or index used to implement a first-fit or near-first-fit policy.
  - This is updated as allocations occur, so the search does not always start from the beginning of
    the heap.
- Counters for total pages allocated and freed:
  - These can be used for accounting, debug, or memory release heuristics.

The page allocator is protected by a single mutex, ensuring that:

- Searching for and marking page ranges occurs atomically.
- The radix tree and bitmaps remain consistent under concurrency.

### 6.3 Arena Integration

The page allocator operates over a global range of page indices that cover all arenas:

- As new arenas are mapped from the OS, the page allocator is informed of the new address range:
  - The allocator’s internal structures are grown to cover the new pages.
  - The pages are initially marked as free and available for allocation.
- Each allocation or free operation references page indices within this global space, regardless of
  which arena the pages live in.

This design allows the heap to grow in chunks of arena size while maintaining a single, uniform
free-space data structure.

---

## 7. Arena and ArenaHints

### 7.1 Arena

An **arena** is a large contiguous region of virtual memory reserved for the heap, typically of a
fixed size per arena instance. For each arena, the system records at least:

- Its base address.
- Its size (often a fixed constant for all arenas).

From this, one can derive:

- The end address.
- Whether a given address falls within the arena.

The arena structure itself may also hold an OS-level mapping object or handle, used to manage the
underlying memory mapping.

### 7.2 ArenaHints

**Arena hints** represent candidate locations for future arenas. The system maintains:

- A collection of addresses or indices where new arenas are likely to be placed.
- A cursor or index indicating which hint to try next.

When the heap needs to grow:

1. The page allocator (under the `GlobalHeap` lock) requests memory using the current hint.
2. If the OS grants memory there, that region is turned into a new arena and added to the heap.
3. The hints are updated to reflect the newly mapped region and potentially new candidate regions
   around it.

This mechanism keeps heap growth predictable and avoids scattering arenas randomly across the
virtual address space.

---

## 8. SizeClass and SpanClass

### 8.1 SizeClass

The heap defines a fixed set of **size classes**:

- Each size class corresponds to a specific object size in bytes.
- A table maps from size class index to size.
- A helper maps from a requested allocation size to the smallest size class that can hold it, or
  returns “none” if the size is too large (in which case the object is treated as a large
  allocation).

Each size class also has an associated **page count**:

- The number of pages allocated for a block of that class.
- This controls how many objects of that size fit in a block.

### 8.2 SpanClass

A **span class** combines:

- A size class.
- A flag indicating whether objects in the span contain pointers (scan) or do not contain pointers
  (noscan).

Encoding:

- Span classes are often represented as a small integer combining:
  - The size class index.
  - A single bit for scan/noscan.

Each span class has:

- An index used to:
  - Select the correct central cache in `GlobalHeap`.
  - Index into the `ThreadHeap` `alloc` array.
- A method to extract its underlying size class.
- A method to tell whether it is scan or noscan.

This design ensures that:

- For each span class, there is exactly one central cache.
- Blocks for that span class are homogeneous with respect to both size and pointer layout.

---

## 9. Allocation and Freeing Flow Summary

### 9.1 Small Allocation

1. The caller (e.g., interpreter or JIT) calls a `ThreadHeap` allocation function with a requested
   size and scan/noscan flag.
2. The `ThreadHeap`:
   - Computes the appropriate size class and span class.
   - Looks up the current block for that span class in its local `alloc` array.
3. If the current block:
   - Has free slots, the allocator:
     - Finds the next free slot using the block’s bitmaps and free index.
     - Marks the slot as allocated.
     - Returns the corresponding pointer.
   - Has no free slots or is missing, the allocator:
     - Calls an internal refill routine which:
       - Locks the central cache for that span class via the `GlobalHeap` reference.
       - Requests a suitable block from the central, which may in turn allocate a new block from the
         page allocator.
       - Installs the new block into the `ThreadHeap`.
     - After refill, the block is used to allocate a slot as above.

### 9.2 Large Allocation

For objects larger than the maximum small size class:

1. A special large-size path is used:
   - Compute the number of pages needed to hold the object.
   - Request a block directly from `GlobalHeap`/page allocator for that exact size.
2. The block is configured to hold a single object of the large size.
3. Allocation returns the base address of the block (plus any object header as appropriate).
4. Freeing a large object eventually frees the entire block back to the page allocator.

### 9.3 Freeing and Sweeping

Freeing and sweeping of blocks is coordinated between:

- Blocks (with their allocation bitmaps and optional mark bits).
- Central caches (which maintain lists of blocks in various states).
- The page allocator (which tracks free pages and merges page ranges).

The typical flow:

1. A block is periodically swept by GC or a sweep routine:
   - It examines allocation and mark bits to find free objects.
   - It may transform an entirely free block into a candidate for return to the page allocator.
2. The block cache (central) decides:
   - Whether to hold the emptied block in its own data structures for reuse.
   - Or return it fully to the page allocator, freeing the underlying pages.
3. The page allocator:
   - Marks the corresponding page range as free.
   - Updates its radix tree summaries to reflect new free capacity.
   - Potentially triggers memory release to the OS as needed.

---

## 10. VM Integration and Lifetime

The expected integration with a VM looks like:

1. The VM creates a `GlobalHeap` instance for each VM instance and wraps it in a reference-counted
   pointer.
2. For each execution context or thread:
   - A `ThreadHeap` is constructed with a clone of the reference to the `GlobalHeap`.
   - The JIT or interpreter stores a pointer to the `ThreadHeap` in its thread context.
3. During execution:
   - All allocations flow through the `ThreadHeap`.
   - The `ThreadHeap` performs fast-path allocations and acquires new blocks from `GlobalHeap` as
     needed.
4. At shutdown:
   - Execution contexts and their `ThreadHeap`s are dropped.
   - Once all references to the `GlobalHeap` are dropped, the heap is destroyed and all heap memory
     can be released.

This structure allows:

- Multiple VMs with independent heaps in the same process.
- Clean teardown of all heap state when a VM terminates, without static global heaps.
