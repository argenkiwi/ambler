require_relative '../ambler'

class Start < Ambler::Step
  def resolve(state)
    loop do
      print "Enter a starting number (or press Enter for default 0): "
      user_input = gets.chomp

      if user_input.empty?
        state[:count] = 0
        puts "Starting count from default: #{state[:count]}"
        return [state, :count]
      end

      initial_count = Integer(user_input)
      state[:count] = initial_count
      puts "Starting count from: #{state[:count]}"
      return [state, :count]
    rescue ArgumentError
      puts "Invalid input. Please enter a valid number or press Enter."
    end
  end
end
