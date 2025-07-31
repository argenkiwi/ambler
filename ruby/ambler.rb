
class Next
  attr_reader :next_func, :state

  def initialize(next_func, state)
    @next_func = next_func
    @state = state
  end

  def call
    @next_func.call(@state)
  end
end

def amble(initial, state)
  nxt = initial.call(state)
  while nxt
    nxt = nxt.call
  end
end
