re = /ruby/
target = "mrubyedge"
if m = (target =~ re)
  puts "matched: #{m}"
end

target2 = "micropython"
if re !~ target2
  puts "not matched"
end