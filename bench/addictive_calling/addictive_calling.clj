(defn increment [x] (+ x 1))
(let [limit 10000000]
  (loop [i 0]
    (when (< i limit)
      (recur (increment i)))))
