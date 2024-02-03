data = {}

function data:extend (t)
    if t.type ~= nil then -- we're extending a single prototype
        rawset(self, rawlen(self)+1, t)
        return
    end

    for _, v in ipairs(t) do
        rawset(self, rawlen(self)+1, v)
    end
end

setmetatable(data, {
    __index = data,
    __newindex = function (t, k, v) end,
})