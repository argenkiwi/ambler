require_relative 'ambler'

# Nodes.
module Node
  START = 1
  STEP = 2
  STOP = 3
end

def start(state)
  puts "Let's count..."
  [state, nil]
end

def step(state)
  count = state + 1
  puts "...#{count}..."
  [count, [true, false].sample]
end

def stop(state)
  puts "...stop."
  [state, nil]
end

# Flow.
def direct(state, node)
  case node
  when Node::START
    resolve(start(state), ->(_) { Node::STEP })
  when Node::STEP
    resolve(step(state), ->(should_continue) { should_continue ? Node::STEP : Node::STOP })
  when Node::STOP
    resolve(stop(state), ->(_) { nil })
  end
end

amble(0, Node::START, method(:direct))
