
def resolve(result, direct)
  state, action = result
  [state, direct.call(action)]
end

def amble(state, edge, follow)
  current_state = state
  current_edge = edge
  while current_edge
    current_state, current_edge = follow.call(current_state, current_edge)
  end
  [current_state, nil]
end
