def count(state)
  puts "Current count: #{state}"
  sleep(1)
  state += 1

  if rand < 0.5
    [state, :count]
  else
    [state, :stop]
  end
end
