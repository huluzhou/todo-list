const { invoke } = window.__TAURI__.core;

/**
 * 从后端加载待办列表，按 order 排序后渲染到 #todo-list
 * @param {HTMLUListElement} listEl - #todo-list 容器
 */
async function loadAndRenderTodos(listEl) {
  const list = await invoke("load_todos");
  const todos = Array.isArray(list) ? list : [];
  const sorted = [...todos].sort((a, b) => (a.order ?? 0) - (b.order ?? 0));

  listEl.innerHTML = "";

  if (sorted.length === 0) {
    const emptyLi = document.createElement("li");
    emptyLi.className = "todo-empty";
    emptyLi.textContent = "暂无待办";
    listEl.appendChild(emptyLi);
    return;
  }

  for (const todo of sorted) {
    const li = document.createElement("li");
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

window.addEventListener("DOMContentLoaded", () => {
  const listEl = document.querySelector("#todo-list");
  if (!listEl) return;
  loadAndRenderTodos(listEl);
});
