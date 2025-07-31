
class Next
  attr_reader :run

  def initialize(&run)
    @run = run
  end
end

def amble(initial, state)
  nxt = initial.call(state)
  while nxt
    nxt = nxt.run.call
  end
end
