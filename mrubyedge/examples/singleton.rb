obj = Object.new

def obj.singleton_hello
  "Hello from singleton class!"
end

puts obj.singleton_hello