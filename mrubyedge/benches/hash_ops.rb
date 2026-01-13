def hash_ops
  h = {}
  
  # 100回挿入
  100.times do |i|
    h["k#{i}"] = "v#{i}"
  end
  
  # 100回取得
  result = []
  100.times do |i|
    result << h["k#{i}"]
  end
  
  result
end

hash_ops
