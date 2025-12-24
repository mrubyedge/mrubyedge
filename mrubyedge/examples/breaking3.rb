def myyield
  yield 1
  yield 1
  yield 1
  yield 1
  yield 1
end

def myyield2
  yield 2
  yield 2
  yield 2
  yield 2
  yield 2
end

def test_break
  x = 0
  y = 0
  myyield do |i|
    puts "loop #{x}, #{y}"
    #__debug__vm_info
    myyield2 do |j|
      y += j
      #__debug__vm_info
      break if y >= 6
    end
    puts "loop #{x}, #{y}"
    #__debug__vm_info
    x += 1
  end
  [x, y]
end

p test_break