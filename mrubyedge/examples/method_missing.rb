class Foo
  def method_missing(name, *args, **kwargs)
    puts "Called #{name} with #{args.inspect}, #{kwargs.inspect}"
  end
end

foo = Foo.new
foo.bar(1, 2, 3, a: 4, b: 5)