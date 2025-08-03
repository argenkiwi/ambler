require_relative '../ambler'

class Stop < Ambler::Step
  def resolve(state)
    puts "Final count: #{state[:count]}"
    [state, nil]
  end
end
