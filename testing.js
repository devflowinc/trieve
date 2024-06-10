const resp = fetch('http://127.0.0.1:8090/api/chunk/search', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
        'TR-Dataset': '0cc04f4b-bd0f-423e-9a7b-ac791856030c',
        'Authorization': 'tr-dsaHswA0zCb7DVONcTMZYBjogrUdJdIB'
    },
    body: JSON.stringify({
        "query": "test",
        "search_type": "semantic",
    })
}).then((response) => {
    console.log(response.status);
}).catch((error) => {
    console.error('Error:', error);
})