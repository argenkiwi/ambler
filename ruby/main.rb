require_relative 'ambler'

def start(state)
  print 'Enter a starting number (or press Enter for default): '
  input = gets.chomp

  if input.empty?
    puts 'Using default starting number.'
    Next.new(method(:count), state)
  elsif input.match?(/^\d+$/)
    Next.new(method(:count), input.to_i)
  else
    puts 'Invalid number, please try again.'
    Next.new(method(:start), state)
  end
end

def count(state)
  puts "Count: #{state}"
  sleep(1)
  new_state = state + 1
  if rand > 0.5
    Next.new(method(:count), new_state)
  else
    Next.new(method(:stop), new_state)
  end
end

def stop(state)
  puts "Stopping count at #{state}."
  nil
end

amble(method(:start), 0)
