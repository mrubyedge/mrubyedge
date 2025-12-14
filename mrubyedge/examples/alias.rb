class Alias1
  def initialize(name)
    @name = name
  end

  def greet
    puts "Hello, #{@name}!"
  end

  alias say_hello greet
end

f = Alias1.new("World of alias")
f.say_hello

class Alias2
  def initialize(name)
    @name = name
  end

  def greet
    puts "Hello, #{@name}!"
  end

  alias say_hello greet
  undef greet
end

f = Alias2.new("World of undef")
f.say_hello