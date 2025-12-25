def outer
  inner do
    inner do
      __debug__vm_info
      return 5471
    end
  end
  puts "unreachable"
  return :unreachable
end

def outer2
  puts "hoge"
  inner do
    :a
  end

  inner do
    inner do
      puts "hoge"
      [:a].size
      1.times do
        2.times do
          __debug__vm_info
          return 5471
        end
      end
    end
  end
  puts "unreachable"
  return :unreachable
end

def outer3
  k = 0
  3.times do |i|
    k += i
    4.times do |j|
      k += j
      __debug__vm_info
      return k if k > 10
    end
  end
  9999
end

def inner
  yield
  puts "unreachable"
  return :unreachable
end

def main
  p outer
end

main