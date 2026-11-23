//! pages of a long log

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Paging {
    pub page: usize,
    pub pages: usize,
    pub from: usize,
    pub to: usize,
    pub total: usize,
}

/// `want` counts from 1; without it the last page (the newest lines) is shown
pub fn paging(total: usize, per: usize, want: Option<usize>) -> Paging {
    let per = per.max(1);
    let pages = total.div_ceil(per).max(1);
    let page = want.unwrap_or(pages).clamp(1, pages);
    let from = (page - 1) * per;
    Paging { page, pages, from, to: (from + per).min(total), total }
}
