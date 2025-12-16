class Test
  def self.hello
    123
  end

  def self.hello2
    10000
  end
end

class Test2 < Test
  def self.hello
    super + 1
  end

  attr_reader :value
end

p Test2.hello
p Test2.hello2