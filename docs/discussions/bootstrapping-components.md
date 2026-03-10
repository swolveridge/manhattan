The prompt generator is an odd one - it's a strange meta-level component.
We only need it because of the specific piece of software we're building - if we were aiming to build something _other_ than ourselves, then we wouldn't be building it most likely.
That raises an odd question about how this might work in the future - if we're building some other piece of software with this system and it needs a specialised component _to build it_.
We're not building ourselves at that point, so it's clearly not right for it to go into _our_ codebase - but it's only in the target codebase for _bootstrapping_.
Maybe that's fine, but it means we sometimes building meta components in the target codebase.
Something I'd not though of before - it's all a bit harder to see when you're building _yourself_.

When building a piece of software, we may first need to build bootstrapping components to _help us build that software_.