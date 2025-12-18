def onetimes
  yield
  puts "whoa?"
end

p(onetimes do
  puts "dummy"
  break 42
end)