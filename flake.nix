{
  description = "Oxide text editor";

  outputs = { self }: {
    defaultPackage.x86_64-linux = self;
  };
}
