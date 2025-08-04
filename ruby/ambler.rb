module Ambler
  class Step
    def resolve(state)
      raise NotImplementedError, "#{self.class} has not implemented method '#{__method__}'"
    end
  end

  def self.amble(state, lead, &follow)
    current_lead = lead
    current_state = state

    while current_lead
      step = follow.call(current_lead)
      current_state, current_lead = step.resolve(current_state)
    end

    current_state
  end
end
