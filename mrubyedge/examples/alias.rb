class For
  def initialize(name)
    @name = name
  end

  def greet
    puts "Hello, #{@name}!"
  end

  alias say_hello greet
  undef greet
end

f = For.new("World")
f.say_hello