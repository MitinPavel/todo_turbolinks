// Setup inline description editing for an existing todo.

function startDescriptionEditing(id) {
  $('.todo-item').removeClass('editing');

  var input = $('#' + id).addClass('editing').
      find("input[name='description']").
      focus().
      get(0);
  input.selectionStart = input.selectionEnd = input.value.length;
}
