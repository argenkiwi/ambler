require_relative '../ambler'

class Count < Ambler::Step
  def resolve(state)
    puts "Current count: #{state[:count]}"
    sleep(1)
    state[:count] += 1

    if rand < 0.5
      [state, :count]
    else
      [state, :stop]
    end
  end
end
