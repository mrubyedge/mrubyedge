test = lambda { puts "Hello, world!" }
test.call

test2 = ->(a) { puts "Hello again! #{a}" }
test2.call("MrubyEdge")

test3 = Proc.new { |a, b| puts "Hello from Proc! #{a}, #{b}" }
test3.call("Foo", "Bar")

value = 10
incrementer = Proc.new { |x| value = x + value }
puts incrementer.call(5)
puts incrementer.call(10)
puts incrementer.call(100)