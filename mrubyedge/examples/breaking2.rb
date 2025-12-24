def puts2(a)
  a
end

ret = 4.times do |i|
  puts "loop #{i}"
  puts2 "dummy"
  #i.inspect
  if i > 1
    __debug__vm_info
    break 9999
  end
end

p ret