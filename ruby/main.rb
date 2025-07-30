require_relative 'ambler'

def prompt_for_number
  loop do
    print 'Enter a starting number: '
    input = gets.chomp
    return input.to_i if input.match?(/^\d+$/)
    puts 'Invalid number, please try again.'
  end
end

def prompt_number_node(_state)
  number = prompt_for_number
  Next.new { start_node(number) }
end

def start_node(state)
  puts "Starting count from #{state}"
  Next.new { step_node(state) }
end

def step_node(state)
  new_state = state + 1
  puts "Count: #{new_state}"
  if rand > 0.5
    Next.new { step_node(new_state) }
  else
    Next.new { stop_node(new_state) }
  end
end

def stop_node(_state)
  puts 'Stopping count.'
  nil
end

amble_from { prompt_number_node(0) }