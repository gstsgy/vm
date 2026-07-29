// vm-gui 前端：纯原生 JS，通过 Tauri 全局 API 调用后端命令（withGlobalTauri）。
if (!window.__TAURI__) {
  document.body.innerHTML =
    '<div style="padding:24px;font-family:sans-serif;color:#c00">初始化失败：Tauri API 未注入（withGlobalTauri 未启用）</div>';
  throw new Error("__TAURI__ missing");
}
const { invoke } = window.__TAURI__;

let state = { categories: {} };
let current = null;
// 编辑模式标记：null = 新增；非 null = 编辑目标
let editingCat = null; // 类目名
let editingVer = null; // 版本号

const $ = (id) => document.getElementById(id);
const status = (msg, kind) => {
  const el = $("status");
  el.textContent = msg;
  el.className = "statusbar" + (kind ? " " + kind : "");
};
const show = (id, on) => $(id).classList.toggle("hidden", !on);

async function refresh() {
  try {
    state = await invoke("get_state");
  } catch (e) {
    status("加载失败: " + e, "err");
    return;
  }
  renderCats();
  if (current && state.categories[current]) {
    renderVersions(current);
  } else {
    current = null;
    show("catPanel", false);
    show("empty", true);
  }
}

function renderCats() {
  const ul = $("catList");
  ul.innerHTML = "";
  const names = Object.keys(state.categories).sort();
  for (const name of names) {
    const li = document.createElement("li");
    li.textContent = name;
    if (name === current) li.classList.add("active");
    li.onclick = () => selectCat(name);
    ul.appendChild(li);
  }
}

function selectCat(name) {
  current = name;
  renderCats();
  renderVersions(name);
  show("empty", false);
  show("catPanel", true);
}

function renderVersions(name) {
  const cat = state.categories[name];
  $("catName").textContent = name;
  $("catDesc").textContent = cat.description || "";
  const ul = $("verList");
  ul.innerHTML = "";
  const versions = Object.keys(cat.versions).sort();
  if (versions.length === 0) {
    const li = document.createElement("li");
    li.innerHTML = '<span class="muted">暂无版本，点击右上角「+ 添加版本」</span>';
    ul.appendChild(li);
  }
  for (const v of versions) {
    const entry = cat.versions[v];
    const li = document.createElement("li");

    const left = document.createElement("div");
    const nameEl = document.createElement("span");
    nameEl.className = "vname";
    nameEl.textContent = v;
    left.appendChild(nameEl);
    if (cat.active === v) {
      const badge = document.createElement("span");
      badge.className = "badge";
      badge.textContent = "当前";
      left.appendChild(badge);
    }
    const pathEl = document.createElement("div");
    pathEl.className = "vpath";
    pathEl.textContent = entry.path + (entry.bin ? "  (bin: " + entry.bin + ")" : "");
    left.appendChild(pathEl);

    const actions = document.createElement("div");
    actions.className = "actions";
    const useBtn = document.createElement("button");
    useBtn.className = "primary sm";
    useBtn.textContent = "切换";
    useBtn.onclick = () => useVersion(name, v);
    const editBtn = document.createElement("button");
    editBtn.className = "ghost sm";
    editBtn.textContent = "编辑";
    editBtn.onclick = () => openEditVersion(name, v);
    const delBtn = document.createElement("button");
    delBtn.className = "danger sm";
    delBtn.textContent = "删除";
    delBtn.onclick = () => removeVersion(name, v);
    actions.appendChild(useBtn);
    actions.appendChild(editBtn);
    actions.appendChild(delBtn);

    li.appendChild(left);
    const spacer = document.createElement("div");
    spacer.className = "spacer";
    li.appendChild(spacer);
    li.appendChild(actions);
    ul.appendChild(li);
  }
}

async function useVersion(cat, ver) {
  try {
    await invoke("use_version", { category: cat, version: ver });
    status(`已切换 ${cat} -> ${ver}`, "ok");
    await refresh();
  } catch (e) {
    status("切换失败: " + e, "err");
  }
}

async function removeVersion(cat, ver) {
  if (!confirm(`确认删除版本 ${ver}？`)) return;
  try {
    await invoke("remove_version", { category: cat, version: ver });
    status(`已删除版本 ${ver}`);
    await refresh();
  } catch (e) {
    status("删除失败: " + e, "err");
  }
}

// ---- 弹窗逻辑 ----
function openModal(id) { show(id, true); }
function closeModals() {
  show("catModal", false);
  show("verModal", false);
}

$("addCatBtn").onclick = () => {
  editingCat = null;
  $("catModalTitle").textContent = "新增类目";
  $("catNameInput").value = ""; $("catNameInput").disabled = false;
  $("catDescInput").value = "";
  openModal("catModal");
};
$("editCatBtn").onclick = () => {
  if (!current) return;
  editingCat = current;
  $("catModalTitle").textContent = "编辑类目";
  $("catNameInput").value = current; $("catNameInput").disabled = true; // 类目名不可改
  $("catDescInput").value = state.categories[current].description || "";
  openModal("catModal");
};
$("delCatBtn").onclick = async () => {
  if (!current) return;
  if (!confirm(`确认删除类目 ${current} 及其全部版本记录？（不会删除真实安装目录）`)) return;
  try {
    await invoke("remove_category", { name: current });
    status(`已删除类目 ${current}`);
    current = null;
    await refresh();
  } catch (e) {
    status("删除类目失败: " + e, "err");
  }
};
$("addVerBtn").onclick = () => {
  if (!current) return;
  editingVer = null;
  $("verModalTitle").textContent = "添加版本";
  $("verInput").value = ""; $("verInput").disabled = false;
  $("pathInput").value = ""; $("binInput").value = "";
  openModal("verModal");
};
function openEditVersion(cat, ver) {
  editingVer = ver;
  const entry = state.categories[cat].versions[ver];
  $("verModalTitle").textContent = "编辑版本";
  $("verInput").value = ver; $("verInput").disabled = true; // 版本号不可改
  $("pathInput").value = entry.path;
  $("binInput").value = entry.bin || "";
  openModal("verModal");
}
document.querySelectorAll("[data-close]").forEach((b) => (b.onclick = closeModals));

$("catSave").onclick = async () => {
  const name = $("catNameInput").value.trim();
  const desc = $("catDescInput").value.trim();
  if (!name) return status("类目名称不能为空", "err");
  try {
    if (editingCat) {
      await invoke("edit_category", { name: editingCat, desc });
      status(`已更新类目 ${editingCat}`, "ok");
    } else {
      await invoke("add_category", { name, desc });
      status(`已新增类目 ${name}`, "ok");
    }
    closeModals();
    await refresh();
  } catch (e) {
    status((editingCat ? "编辑" : "新增") + "类目失败: " + e, "err");
  }
};

$("verSave").onclick = async () => {
  const version = $("verInput").value.trim();
  const path = $("pathInput").value.trim();
  const bin = $("binInput").value.trim() || null;
  if (!version || !path) return status("版本号与路径必填", "err");
  try {
    if (editingVer) {
      await invoke("edit_version", { category: current, version: editingVer, path, bin });
      status(`已更新版本 ${editingVer}`, "ok");
    } else {
      await invoke("add_version", { category: current, version, path, bin });
      status(`已添加版本 ${version}`, "ok");
    }
    closeModals();
    await refresh();
  } catch (e) {
    status((editingVer ? "编辑" : "添加") + "版本失败: " + e, "err");
  }
};

$("envBtn").onclick = async () => {
  try {
    const s = await invoke("env_snippet");
    const ta = document.createElement("textarea");
    ta.value = s;
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    document.body.removeChild(ta);
    status("PATH 配置已复制到剪贴板", "ok");
  } catch (e) {
    status("获取 PATH 失败: " + e, "err");
  }
};

refresh();
