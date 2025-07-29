main :: IO ()
main = loop 0
  where
    loop i
      | i < 5000000 = loop (i + 1)
      | otherwise = return ()
