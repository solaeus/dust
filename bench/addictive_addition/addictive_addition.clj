(let [limit 5000000]
  (loop [i 0]
    (when (< i limit)
      (recur (inc i)))))
