# Tower of Hanoi
# Move disks from source to destination using auxiliary peg

def hanoi(n, source, destination, auxiliary, step)
  if n == 1
    step[0] += 1
    puts "Step #{step[0]}: Move disk 1 from #{source} to #{destination}"
    return
  end
  
  hanoi(n - 1, source, auxiliary, destination, step)
  step[0] += 1
  puts "Step #{step[0]}: Move disk #{n} from #{source} to #{destination}"
  hanoi(n - 1, auxiliary, destination, source, step)
end

puts "Tower of Hanoi with 3 disks:"
puts "----------------------------"
hanoi(3, 'A', 'C', 'B', [0])
puts "----------------------------"
puts "Complete!"