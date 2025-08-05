def start(state)
  loop do
    print "Enter a starting number (or press Enter for default 0): "
    user_input = gets.chomp

    if user_input.empty?
      puts "Starting count from default: #{state}"
      return [state, :count]
    end

    initial_count = Integer(user_input)
    state = initial_count
    puts "Starting count from: #{state}"
    return [state, :count]
  rescue ArgumentError
    puts "Invalid input. Please enter a valid number or press Enter."
  end
end
