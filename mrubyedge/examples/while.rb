a = 0
while true
  begin
    a += 1
    break if a > 10
  ensure
    puts "ensure: a=#{a}"
  end
end
p a