const { invoke } = window.__TAURI__.core;

/** 当前内存中的待办列表，load 后赋值，增删改后更新再 save_todos */
let todos = [];

/** 当前是否始终置顶（与 window.json 一致，默认 true；后端无查询接口，由前端记录） */
let alwaysOnTop = true;

/**
 * 按 order 排序后的待办数组（不修改原数组）
 * @param {Array<{ id: string, text: string, done: boolean, order: number }>} list
 * @returns {Array}
 */
function sortedTodos(list) {
  return [...(list || [])].sort((a, b) => (a.order ?? 0) - (b.order ?? 0));
}

/**
 * 将待办数组渲染到 #todo-list
 * @param {HTMLUListElement} listEl - #todo-list 容器
 * @param {Array<{ id: string, text: string, done: boolean, order: number }>} list - 已排序的待办列表
 */
function renderTodos(listEl, list) {
  listEl.innerHTML = "";

  if (!list || list.length === 0) {
    const emptyLi = document.createElement("li");
    emptyLi.className = "todo-empty";
    emptyLi.textContent = "暂无待办";
    listEl.appendChild(emptyLi);
    return;
  }

  for (const todo of list) {
    const li = document.createElement("li");
    li.dataset.id = todo.id;
    if (todo.done) li.classList.add("done");

    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.checked = !!todo.done;
    checkbox.dataset.id = todo.id;

    const textSpan = document.createElement("span");
    textSpan.className = "todo-text";
    textSpan.textContent = todo.text ?? "";

    const deleteBtn = document.createElement("button");
    deleteBtn.type = "button";
    deleteBtn.className = "todo-delete";
    deleteBtn.textContent = "删除";
    deleteBtn.dataset.id = todo.id;

    li.appendChild(checkbox);
    li.appendChild(textSpan);
    li.appendChild(deleteBtn);
    listEl.appendChild(li);
  }
}

/**
 * 调用 save_todos 持久化当前 todos，成功后重渲染；失败则提示
 * @param {HTMLUListElement} listEl - #todo-list 容器
 */
async function saveTodosAndRender(listEl) {
  try {
    await invoke("save_todos", { todos });
    renderTodos(listEl, sortedTodos(todos));
  } catch (e) {
    console.error("save_todos failed:", e);
    alert("保存失败，请重试");
  }
}

/**
 * 从后端加载待办并赋给内存列表，再渲染
 * @param {HTMLUListElement} listEl - #todo-list 容器
 */
async function loadAndRenderTodos(listEl) {
  try {
    const list = await invoke("load_todos");
    todos = Array.isArray(list) ? list : [];
    renderTodos(listEl, sortedTodos(todos));
  } catch (e) {
    console.error("load_todos failed:", e);
    todos = [];
    renderTodos(listEl, []);
  }
}

/**
 * 添加待办：校验非空，生成 id/order，追加到 todos，save_todos，重渲染，清空输入
 * @param {HTMLFormElement} form - #add-todo-form
 * @param {HTMLUListElement} listEl - #todo-list
 */
async function handleAddTodo(form, listEl) {
  const input = form.querySelector("#add-todo-input");
  const text = (input?.value ?? "").trim();
  if (!text) return;

  const newTodo = {
    id: crypto.randomUUID(),
    text,
    done: false,
    order: todos.length,
  };
  todos.push(newTodo);
  await saveTodosAndRender(listEl);
  if (input) input.value = "";
}

/**
 * 删除待办：从 todos 中按 id 移除，save_todos，重渲染
 * @param {string} id - todo.id
 * @param {HTMLUListElement} listEl - #todo-list
 */
async function handleDeleteTodo(id, listEl) {
  todos = todos.filter((t) => t.id !== id);
  await saveTodosAndRender(listEl);
}

/**
 * 开始行内编辑：该 li 内 .todo-text 变为 input，焦点并选中
 * @param {HTMLLIElement} li - 当前列表项
 * @param {string} currentText - 当前文案
 * @param {HTMLUListElement} listEl - #todo-list 容器
 */
function startEdit(li, currentText, listEl) {
  const textSpan = li.querySelector(".todo-text");
  if (!textSpan || textSpan.tagName === "INPUT") return;

  const input = document.createElement("input");
  input.type = "text";
  input.className = "todo-edit-input";
  input.value = currentText;
  input.dataset.id = li.dataset.id;

  const cancelEdit = () => {
    renderTodos(listEl, sortedTodos(todos));
  };

  input.addEventListener("blur", () => {
    const id = li.dataset.id;
    const item = todos.find((t) => t.id === id);
    const originalText = (item?.text ?? "").trim();
    const newText = (input.value ?? "").trim();
    if (newText !== originalText) {
      if (item) item.text = newText || item.text;
      saveTodosAndRender(listEl);
    } else {
      cancelEdit();
    }
  });

  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      input.blur();
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelEdit();
    }
  });

  textSpan.replaceWith(input);
  input.focus();
  input.select();
}

/**
 * 勾选/取消勾选：更新该项 done，save_todos，重渲染（li 的 class done 由 render 根据 done 设置）
 * @param {string} id - todo.id
 * @param {boolean} done - 新的完成状态
 * @param {HTMLUListElement} listEl - #todo-list
 */
async function handleToggleDone(id, done, listEl) {
  const item = todos.find((t) => t.id === id);
  if (item) {
    item.done = !!done;
    await saveTodosAndRender(listEl);
  }
}

/**
 * 根据当前置顶状态更新 #btn-pin 的文案与样式
 * @param {HTMLButtonElement} btn - #btn-pin 元素
 * @param {boolean} enabled - 是否置顶
 */
function updatePinButton(btn, enabled) {
  if (!btn) return;
  btn.textContent = enabled ? "已置顶" : "置顶";
  btn.classList.toggle("is-pinned", !!enabled);
}

window.addEventListener("DOMContentLoaded", () => {
  const listEl = document.querySelector("#todo-list");
  const form = document.querySelector("#add-todo-form");
  const titleBarDrag = document.querySelector(".title-bar-drag");
  const btnPin = document.querySelector("#btn-pin");
  if (!listEl) return;

  // 标题栏拖动：data-tauri-drag-region 在 Tauri 2 无边框下可能已生效，此处 mousedown+invoke 作为兼容/兜底
  if (titleBarDrag) {
    titleBarDrag.addEventListener("mousedown", () => {
      invoke("start_dragging").catch((e) => console.error("start_dragging failed:", e));
    });
  }

  // 置顶按钮：点击取反状态，调用后端并更新按钮
  if (btnPin) {
    updatePinButton(btnPin, alwaysOnTop);
    btnPin.addEventListener("click", async () => {
      alwaysOnTop = !alwaysOnTop;
      try {
        await invoke("set_always_on_top", { enabled: alwaysOnTop });
        updatePinButton(btnPin, alwaysOnTop);
      } catch (e) {
        console.error("set_always_on_top failed:", e);
        alwaysOnTop = !alwaysOnTop;
        updatePinButton(btnPin, alwaysOnTop);
      }
    });
  }

  // 开机启动开关：加载时同步状态，变更时调用后端
  const autostartCheckbox = document.querySelector("#autostart-checkbox");
  if (autostartCheckbox) {
    // 加载时同步状态
    invoke("is_autostart_enabled")
      .then((enabled) => {
        autostartCheckbox.checked = !!enabled;
      })
      .catch(() => {
        autostartCheckbox.checked = false;
      });
    
    // 变更时调用后端，成功后重新查询状态以确保同步
    let isUpdating = false; // 防止重复触发
    autostartCheckbox.addEventListener("change", async (e) => {
      // 如果正在更新，忽略此次事件
      if (isUpdating) {
        return;
      }
      
      // 阻止事件冒泡，避免可能的干扰
      e.stopPropagation();
      e.preventDefault();
      
      const enabled = autostartCheckbox.checked;
      console.log("开机启动状态变更:", enabled);
      
      isUpdating = true;
      try {
        await invoke("set_autostart", { enabled });
        console.log("set_autostart 调用成功");
        
        // 设置成功后重新查询状态以确保复选框与实际状态同步
        // 添加短暂延迟，确保注册表写入完成
        await new Promise(resolve => setTimeout(resolve, 200));
        
        const actualEnabled = await invoke("is_autostart_enabled");
        console.log("查询到的实际状态:", actualEnabled);
        
        // 使用 requestAnimationFrame 确保 DOM 更新在下一帧
        requestAnimationFrame(() => {
          autostartCheckbox.checked = !!actualEnabled;
          if (actualEnabled !== enabled) {
            console.warn("开机启动状态不一致: 期望", enabled, "实际", actualEnabled);
            if (!actualEnabled && enabled) {
              alert("设置开机启动失败，请检查是否有足够的权限");
            }
          }
          isUpdating = false;
        });
      } catch (e) {
        console.error("set_autostart failed:", e);
        // 失败时回退到之前的状态
        requestAnimationFrame(() => {
          autostartCheckbox.checked = !enabled;
          isUpdating = false;
        });
        // 显示更友好的错误提示
        const errorMsg = String(e);
        if (errorMsg.includes("权限") || errorMsg.includes("拒绝访问") || errorMsg.includes("os error 5")) {
          alert("设置开机启动失败：权限不足\n\n请尝试：\n1. 以管理员身份运行应用\n2. 检查防病毒软件是否阻止了注册表访问\n3. 稍后重试");
        } else {
          alert("设置开机启动失败: " + errorMsg);
        }
      }
    });
  }

  loadAndRenderTodos(listEl);

  // 添加：表单 submit 或添加按钮点击
  if (form) {
    form.addEventListener("submit", (e) => {
      e.preventDefault();
      handleAddTodo(form, listEl);
    });
  }

  // 删除、编辑、勾选：事件委托到 #todo-list
  listEl.addEventListener("click", (e) => {
    const deleteBtn = e.target.closest(".todo-delete");
    const textSpan = e.target.closest(".todo-text");
    if (deleteBtn) {
      const id = deleteBtn.dataset.id;
      if (id) handleDeleteTodo(id, listEl);
      return;
    }
    if (textSpan && textSpan.tagName !== "INPUT") {
      const li = textSpan.closest("li");
      const id = li?.dataset?.id;
      const item = id ? todos.find((t) => t.id === id) : null;
      if (li && item) startEdit(li, item.text ?? "", listEl);
    }
  });

  listEl.addEventListener("change", (e) => {
    if (e.target.type === "checkbox") {
      const id = e.target.dataset.id;
      if (id) handleToggleDone(id, e.target.checked, listEl);
    }
  });
});
