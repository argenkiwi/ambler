require_relative './ambler'
require_relative './step/start'
require_relative './step/count'
require_relative './step/stop'

initial_state = 0

Ambler.amble(initial_state, :start) do |lead, state|
  case lead
  when :start
    start(state)
  when :count
    count(state)
  when :stop
    stop(state)
  end
end
