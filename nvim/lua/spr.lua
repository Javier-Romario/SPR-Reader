local M = {}

M.defaults = {
  bin = "spr",
  wpm = 300,
  inline = true,
  preview_words = 0,
  window = {
    width = 42, -- columns (>1) or fraction of editor width (<=1); matches spr's box (40 content + 2 border)
    height = 5, -- rows (>1) or fraction of editor height (<=1); spr inline viewport is 5 rows
    border = "none",
  },
}

local function deepcopy(v)
  if type(v) ~= "table" then
    return v
  end
  local out = {}
  for k, val in pairs(v) do
    out[deepcopy(k)] = deepcopy(val)
  end
  return out
end

local function merge(t, d)
  local out = deepcopy(d)
  for k, v in pairs(t or {}) do
    if type(v) == "table" and type(d[k]) == "table" then
      out[k] = merge(v, d[k])
    else
      out[k] = v
    end
  end
  return out
end

-- Exposed for tests
M._merge = merge

--- Pure: extract selection text from raw lines + visual mode + 1-indexed cols.
--- Exposed for testing.
---@param lines string[]
---@param mode string  "v" (charwise), "V" (linewise), "\22" (blockwise)
---@param cscol integer  1-indexed start column
---@param cecol integer  1-indexed end column
---@return string|nil
function M.extract(lines, mode, cscol, cecol)
  local n = #lines
  if n == 0 then
    return nil
  end

  if mode == "V" then
    return table.concat(lines, "\n")
  elseif mode == "\22" then -- blockwise Ctrl-v
    local out = {}
    for i, l in ipairs(lines) do
      out[i] = string.sub(l, cscol, cecol)
    end
    return table.concat(out, "\n")
  end

  -- charwise
  if n == 1 then
    return string.sub(lines[1], cscol, cecol)
  end
  local out = { string.sub(lines[1], cscol) }
  for i = 2, n - 1 do
    out[i] = lines[i]
  end
  out[n] = string.sub(lines[n], 1, cecol)
  return table.concat(out, "\n")
end

--- Extract current visual selection (handles charwise, linewise, blockwise).
---@return string|nil
local function get_visual_selection()
  local _, csrow, cscol = unpack(vim.fn.getpos("'<"))
  local _, cerow, cecol = unpack(vim.fn.getpos("'>"))
  local mode = vim.fn.visualmode()

  if csrow == 0 or cerow == 0 then
    return nil
  end

  local lines = vim.fn.getline(csrow, cerow)
  return M.extract(lines, mode, cscol, cecol)
end

--- Open a floating terminal running `cmd`, invoke `on_exit` when it closes.
---@param cmd string[]
---@param on_exit fun()|nil
---@param win_opts table
local function open_term(cmd, on_exit, win_opts)
  local buf = vim.api.nvim_create_buf(false, true) -- scratch, unlisted

  -- Size the float relative to the current window, not the whole editor,
  -- so it matches the parent split/panel it was invoked from. Values <= 1
  -- are treated as fractions of the window; values > 1 are absolute cells.
  local curwin = vim.api.nvim_get_current_win()
  local win_w = vim.api.nvim_win_get_width(curwin)
  local win_h = vim.api.nvim_win_get_height(curwin)

  local function dim(value, total)
    if value <= 1 then
      return math.floor(total * value)
    end
    return math.floor(value)
  end

  local width = dim(win_opts.width, win_w)
  local height = dim(win_opts.height, win_h)
  width = math.max(16, math.min(width, win_w - 2))
  height = math.max(5, math.min(height, win_h - 2))
  local row = math.floor((win_h - height) / 2)
  local col = math.floor((win_w - width) / 2)

  local win = vim.api.nvim_open_win(buf, true, {
    relative = "win",
    win = curwin,
    width = width,
    height = height,
    row = row,
    col = col,
    style = "minimal",
    border = win_opts.border,
  })

  vim.fn.termopen(cmd, {
    on_exit = function()
      if vim.api.nvim_win_is_valid(win) then
        vim.api.nvim_win_close(win, true)
      end
      if vim.api.nvim_buf_is_valid(buf) then
        vim.api.nvim_buf_delete(buf, { force = true })
      end
      if on_exit then
        on_exit()
      end
    end,
  })
  vim.cmd("startinsert")
end

--- Build the `spr` argv for a given text and options.
---@param text string
---@param opts table
---@return string[] args, string tmpfile
local function build_cmd(text, opts)
  local tmp = vim.fn.tempname() .. ".txt"
  local f = assert(io.open(tmp, "w"))
  f:write(text)
  f:close()

  local args = { opts.bin, "--file", tmp, "--wpm", tostring(opts.wpm) }
  if opts.inline then
    table.insert(args, "--inline")
  end
  if opts.preview_words and opts.preview_words > 0 then
    table.insert(args, "--preview-words")
    table.insert(args, tostring(opts.preview_words))
  end
  return args, tmp
end

--- Speed-read `text` in a floating terminal.
---@param text string
---@param opts? table
function M.read(text, opts)
  opts = merge(opts, M.defaults)

  if not opts.bin or vim.fn.executable(opts.bin) ~= 1 then
    vim.notify("SPR: binary not found: " .. tostring(opts.bin), vim.log.levels.ERROR)
    return
  end

  if not text or text:match("^%s*$") then
    vim.notify("SPR: nothing to read", vim.log.levels.WARN)
    return
  end

  local args, tmp = build_cmd(text, opts)
  open_term(args, function()
    os.remove(tmp)
  end, opts.window)
end

--- Speed-read the current visual selection.
---@param opts? table
function M.selection(opts)
  local text = get_visual_selection()
  if not text or text:match("^%s*$") then
    vim.notify("SPR: no text selected", vim.log.levels.WARN)
    return
  end
  M.read(text, opts)
end

--- Speed-read the entire current buffer.
---@param opts? table
function M.buffer(opts)
  local text = table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), "\n")
  M.read(text, opts)
end

--- Speed-read the current line.
---@param opts? table
function M.line(opts)
  M.read(vim.api.nvim_get_current_line(), opts)
end

return M
