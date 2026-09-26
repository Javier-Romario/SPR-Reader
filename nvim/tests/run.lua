-- Minimal standalone test harness + SPR plugin tests.
-- No dependencies: run with `lua nvim/tests/run.lua` (or luajit).
-- spr.lua only touches `vim` inside functions, so it loads clean here.

local spr = dofile("nvim/lua/spr.lua")

local tests = {}
local current = {}

local function describe(name, fn)
  current.suite = name
  fn()
  current.suite = nil
end

local function it(name, fn)
  table.insert(tests, { suite = current.suite, name = name, fn = fn })
end

local assert = {}

function assert.is_equal(actual, expected, msg)
  if actual ~= expected then
    error(string.format(
      "%s\n  expected: %s\n  actual:   %s",
      msg or "is_equal failed",
      tostring(expected),
      tostring(actual)
    ), 2)
  end
end

function assert.is_nil(actual, msg)
  if actual ~= nil then
    error(string.format("%s\n  expected nil, got: %s", msg or "is_nil failed", tostring(actual)), 2)
  end
end

function assert.is_truthy(actual, msg)
  if not actual then
    error(string.format("%s\n  expected truthy, got: %s", msg or "is_truthy failed", tostring(actual)), 2)
  end
end

-- Fixtures
local T = {
  "the quick brown fox",
  "jumps over the lazy dog",
  "and the cow jumped over",
}

describe("extract", function()
  it("returns nil for no lines", function()
    assert.is_nil(spr.extract({}, "v", 1, 1))
  end)

  it("charwise single line", function()
    assert.is_equal(spr.extract({ T[1] }, "v", 5, 9), "quick")
  end)

  it("charwise multi line trims first and last", function()
    local got = spr.extract({ T[1], T[2], T[3] }, "v", 5, 13)
    local want = "quick brown fox\njumps over the lazy dog\nand the cow j"
    assert.is_equal(got, want)
  end)

  it("linewise returns whole lines", function()
    assert.is_equal(spr.extract(T, "V", 1, 1), table.concat(T, "\n"))
  end)

  it("blockwise extracts rectangle", function()
    local got = spr.extract({ T[1], T[2], T[3] }, "\22", 1, 3)
    assert.is_equal(got, "the\njum\nand")
  end)

  it("charwise full line single", function()
    assert.is_equal(spr.extract({ T[1] }, "v", 1, #T[1]), T[1])
  end)
end)

describe("merge", function()
  it("defaults used when opts empty", function()
    assert.is_truthy(spr._merge)
    local out = spr._merge(nil, { a = 1 })
    assert.is_equal(out.a, 1)
  end)

  it("scalar overrides default", function()
    local out = spr._merge({ a = 2 }, { a = 1 })
    assert.is_equal(out.a, 2)
  end)

  it("nested tables merge recursively", function()
    local out = spr._merge({ w = { height = 0.9 } }, { w = { height = 0.6, border = "rounded" } })
    assert.is_equal(out.w.height, 0.9)
    assert.is_equal(out.w.border, "rounded")
  end)

  it("does not mutate defaults", function()
    local d = { w = { height = 0.6 } }
    spr._merge({ w = { height = 0.9 } }, d)
    assert.is_equal(d.w.height, 0.6)
  end)
end)

-- Run
local passed, failed = 0, 0
for _, t in ipairs(tests) do
  local label = (t.suite and t.suite .. " > " or "") .. t.name
  local ok, err = pcall(t.fn)
  if ok then
    passed = passed + 1
    io.write("ok - ", label, "\n")
  else
    failed = failed + 1
    io.write("FAIL - ", label, "\n", err, "\n")
  end
end

io.write(string.format("\n%d passed, %d failed\n", passed, failed))
os.exit(failed == 0 and 0 or 1)
