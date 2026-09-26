-- SPR neovim integration: expose commands + default keymaps.

if vim.g.loaded_spr then
  return
end
vim.g.loaded_spr = 1

vim.api.nvim_create_user_command("SprRead", function()
  require("spr").selection()
end, {
  range = true,
  desc = "SPR speed-read the visual selection",
})

vim.api.nvim_create_user_command("SprReadBuffer", function()
  require("spr").buffer()
end, {
  desc = "SPR speed-read the entire buffer",
})

vim.api.nvim_create_user_command("SprReadLine", function()
  require("spr").line()
end, {
  desc = "SPR speed-read the current line",
})

if not vim.g.spr_no_default_maps then
  vim.keymap.set("x", "<leader>sr", require("spr").selection, { desc = "SPR: read selection" })
  vim.keymap.set("n", "<leader>sb", require("spr").buffer, { desc = "SPR: read buffer" })
  vim.keymap.set("n", "<leader>sl", require("spr").line, { desc = "SPR: read line" })
end
