$a1 = [0, 1, 2]
$a2 = [0, 2, 4, 6, 8]

def test_break
  x = 0
  y = 0
  3.times do |i|
    puts "loop #{x}, #{y}"
    #__debug__vm_info
    $a2.each do |j|
      y += j
      puts "  inner loop #{x}, #{y}"
      #__debug__vm_info
      break if j >= 5
    end
    puts "loop #{x}, #{y}"
    #__debug__vm_info
    x += i
  end
  [x, y]
end

p test_break