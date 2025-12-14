re = /ruby/
target = "mrubyedge"
if m = (target =~ re)
  puts "matched: #{m}"
end

target2 = "micropython"
if re !~ target2
  puts "not matched"
end

re3 = /(m?ruby).*?(m?ruby).*?(m?ruby(?:ists)?)/
target3 = "mruby/edge is a mruby for embedded systems, built for rubyists."
matched = re3.match(target3)
if matched
  puts "matched: #{matched[0]}"
  puts "matched: #{matched[1]}"
  puts "matched: #{matched[2]}"
  puts "matched: #{matched[3]}"
end