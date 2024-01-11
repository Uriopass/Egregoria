data = {}

function data:extend (t)
    for _, v in ipairs(t) do
        rawset(self, rawlen(self)+1, v)
    end
end

setmetatable(data, {
    __index = data,
    __newindex = function (t, k, v) end,
})