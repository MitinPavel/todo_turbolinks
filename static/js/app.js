// Setup inline description editing for an existing todo.

function startDescriptionEditing(id) {
  $('.todo-item').removeClass('editing');

  var input = $('#' + id).addClass('editing').
      find("input[name='description']").
      focus().
      get(0);
  input.selectionStart = input.selectionEnd = input.value.length;
}

// Submit forms via AJAX.

function submitForm(form) {
  $.ajax({
    url:      form.action,
    type:     form.method,
    dataType: 'script',
    data:     $(form).serialize()
  });
}

document.addEventListener("turbolinks:load", function() {
  $('form').submit(function (e) {
     e.preventDefault();
     submitForm(e.target);
  });
});
