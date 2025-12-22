ret = 4.times do |i|
  puts "loop #{i}"
  if i > 1
    break 9999
  end
end

p ret