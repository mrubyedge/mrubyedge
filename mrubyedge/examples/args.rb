def splat_it(x, *args)
  args.each do |arg|
    p arg
  end
end

splat_it(10, 20, 30)