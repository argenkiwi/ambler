
class Next
  attr_reader :run

  def initialize(&run)
    @run = run
  end
end

def amble(initial)
  nxt = initial
  while nxt
    nxt = nxt.run.call
  end
end

def amble_from(&initial)
  amble(Next.new(&initial))
end
