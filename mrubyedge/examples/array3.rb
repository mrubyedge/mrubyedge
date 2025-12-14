array = [1, 2]
a1, a2 = *array
p a1  # => 1
p a2  # => 2

array2 = %i(foo bar buz quz)
a11, a12, *rest = *array2
p a11  # => :foo
p a12  # => :bar
p rest  # => [:buz, :quz]