class Router
  def self.get path, &block
    puts "Registered GET #{path}"
    @routes ||= {}
    @routes[path] = block
  end

  def self.request path
    if @routes && @routes[path]
      @routes[path].call(path)
    else
      puts "No route for #{path}"
    end
  end
end

Router.get "/home" do |path|
  puts "Inside /home route: #{path}"
end

Router.request "/home"
Router.request "/about"