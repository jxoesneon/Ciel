def paginate(items, size, page):
    start = page * size
    return items[start:start + size]
