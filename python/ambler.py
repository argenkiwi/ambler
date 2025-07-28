def amble(state, node, step):
    current_state = state
    current_node = node
    while current_node is not None:
        current_state, current_node = step(current_state, current_node)
    return current_state, None
