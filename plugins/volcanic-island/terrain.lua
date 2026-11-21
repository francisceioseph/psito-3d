-- A volcanic island: a cone that rises from the sea, with a crater on top
-- and, optionally, gullies running down its sides.
--
-- x and y go from -1 to 1 across the terrain; return a height from 0 to 1.

local noise = psito.noise
local m = psito.math

return {
  height = function(x, y, p)
    local d = math.sqrt(x * x + y * y)
    -- A wobbly coast, so the island is not a perfect circle.
    d = d + 0.08 * noise.fbm(x * 3, y * 3, { octaves = 3 })

    -- The cone: steep near the top, flatter near the shore.
    local cone = (1 - m.smoothstep(0.0, 0.85, d)) ^ 1.8

    -- Gullies: ridged noise stretched around the peak.
    if p.gullies then
      local angle = math.atan(y, x)
      local g = noise.ridged(angle * 4, d * 6, { octaves = 4 })
      cone = cone * (0.85 + 0.15 * g)
    end

    -- The crater: carve the top away inside its radius.
    local h = cone * p.height
    if d < p.crater then
      h = h - (p.crater - d) * 1.6 * p.height
    end

    return m.clamp(h + 0.04 * noise.fbm(x * 12, y * 12), 0, 1)
  end,
}
