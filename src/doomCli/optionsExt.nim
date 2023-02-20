import std/options


template getOrElse*[T](opt: Option[T], otherwise: untyped): untyped =
  if opt.isSome:
    opt.get
  else:
    otherwise
