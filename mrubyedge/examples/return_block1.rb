def outer
  inner do
    inner do
      return 5471
    end
  end
  puts "unreachable"
  return :unreachable
end

def outer2
  puts "start outer2"
  1.times do
    inner do
      puts "start inner"
      return 5472
    end
    puts "unreachable: after inner"
  end
  puts "unreachable: after times"
end

def outer3
  k = 0
  [0, 1, 2].each do |i|
    k += i
    4.times do |j|
      k += j
      #__debug__vm_info
      return k if k > 10
    end
  end
  9999
end

def outer4
  puts "start outer4"
  inner do
    1.times do
      puts "start times"
      return 5474
    end
    puts "unreachable: after times"
  end
  puts "unreachable: after inner"
end

def outer5
  puts "start outer5"
  inner do
    1.times do
      puts "start times"
      1.times do
        puts "start inner inner"
        return 5475
        puts "unreachable: after return in inner inner"
      end
    end
    puts "unreachable: after times"
  end
  puts "unreachable: after inner"
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
  puts "=== outer3 ==="
  puts outer3
  puts "=== outer4 ==="
  puts outer4
  puts "=== outer5 ==="
  puts outer5 #???
end

main