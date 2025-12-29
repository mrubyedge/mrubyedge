def outer
  puts "start outer"
  inner do
    return 5471
    puts "unreachable inner"
  end
  puts "unreachable outer"
  :unreachable
end

def outer2
  puts "start outer2"
  1.times do
    puts "start times"
    return 5472
    puts "unreachable: after times"
  end
  puts "unreachable outer2"
end

def inner
  yield
  puts "unreachable"
  return :unreachable
end

def main
  puts "=== outer ==="
  puts outer
  puts "=== outer2 ==="
  puts outer2
end

main