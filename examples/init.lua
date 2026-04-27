--- @return Custom
function getCustom() return {} end

--- @type Custom
local c = getCustom()

if c._variant == "B" then
    ---@cast c CustomB
    print(c.COUNT)
end

print(c.COUNT)

print("Hello world!")
