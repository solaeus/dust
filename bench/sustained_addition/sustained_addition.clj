(let [limit 100000000
      result (loop [i 0]
               (if (< i limit)
                 (recur (inc i))
                 i))]
  (println result))
