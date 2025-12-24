def onetimes
  yield
  puts "whoa?"
end

p(onetimes do
  puts "dummy"
  __debug__vm_info
  break 42
end)