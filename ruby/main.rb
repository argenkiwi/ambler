require_relative './ambler'
require_relative './step/start'
require_relative './step/count'
require_relative './step/stop'

initial_state = { count: 0 }

Ambler.amble(initial_state, :start) do |lead|
  case lead
  when :start
    Start.new
  when :count
    Count.new
  when :stop
    Stop.new
  end
end