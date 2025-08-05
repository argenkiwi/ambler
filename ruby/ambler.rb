module Ambler
  def self.amble(state, lead, &follow)
    current_lead = lead
    current_state = state

    while current_lead
      current_state, current_lead = follow.call(current_lead, current_state)
    end

    current_state
  end
end
